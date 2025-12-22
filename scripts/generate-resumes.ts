/* eslint-disable no-console */
/* eslint-disable unicorn/prefer-top-level-await */
/* eslint-disable security/detect-non-literal-fs-filename */
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'

import React from 'react'

import { createFormatter, createTranslator, type Messages } from 'next-intl'

import { renderToBuffer } from '@react-pdf/renderer'
import { plainAddPlaceholder } from '@signpdf/placeholder-plain'
import { P12Signer } from '@signpdf/signer-p12'
import signpdf from '@signpdf/signpdf'
import forge from 'node-forge'

import { ResumePDFDocument } from '@/components/resume/resume-pdf-document'
import { routing } from '@/i18n/routing'
import { siteConfig } from '@/lib/config'
import type { Translations } from '@/types/i18n'

async function getSecret(name: string): Promise<string | undefined> {
  // eslint-disable-next-line security/detect-object-injection
  const environmentValue: string | undefined = process.env[name]
  if (environmentValue !== undefined && environmentValue !== '') {
    return environmentValue
  }

  try {
    const secretPath: string = path.join('/run/secrets', name.toLowerCase())
    const secretValue: string = await readFile(secretPath, 'utf8')
    return secretValue.trim()
  } catch {
    return undefined
  }
}

// eslint-disable-next-line max-lines-per-function
async function exportCertificateAssets(
  signingKeys: Buffer,
  signingCertPassword: string | undefined,
  publicDirectory: string
): Promise<void> {
  console.log('✓ Found signing certificate')

  // Extract public certificate and fingerprint
  const p12Asn1: forge.asn1.Asn1 = forge.asn1.fromDer(
    signingKeys.toString('binary')
  )
  const pkcs12: forge.pkcs12.Pkcs12Pfx = forge.pkcs12.pkcs12FromAsn1(
    p12Asn1,
    signingCertPassword
  )

  // Use string literal for OID to avoid type issues if types are loose
  // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access
  const certBagType: string | undefined = forge.pki.oids['certBag' as any]
  if (certBagType === undefined) {
    return
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const bags: any = pkcs12.getBags({ bagType: certBagType })
  // eslint-disable-next-line security/detect-object-injection, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-assignment
  const certBag: forge.pkcs12.Bag | undefined = bags[certBagType]?.[0]
  const cert: forge.pki.Certificate | undefined = certBag?.cert

  if (cert === undefined) {
    return
  }

  try {
    // Save public certificate
    const pem: string = forge.pki.certificateToPem(cert)
    await writeFile(path.join(publicDirectory, 'certificate.crt'), pem)
    console.log('✓ Exported public certificate')

    // Calculate and save fingerprint
    const der: forge.asn1.Asn1 = forge.pki.certificateToAsn1(cert)
    const messageDigest: forge.md.MessageDigest = forge.md.sha256.create()
    messageDigest.update(forge.asn1.toDer(der).getBytes())
    const fingerprint: string | undefined = messageDigest
      .digest()
      .toHex()
      .match(/.{1,2}/g)
      ?.join(':')
      .toUpperCase()

    if (fingerprint !== undefined) {
      const fingerprintData: { fingerprint: string } = { fingerprint }
      // Write to public directory only
      await writeFile(
        path.join(process.cwd(), 'public/resume-fingerprint.json'),
        JSON.stringify(fingerprintData, null, 2)
      )
      console.log('✓ Exported certificate fingerprint to public')
    }
  } catch (fileError) {
    console.error('⚠ Failed to export certificate asset:', fileError)
  }
}

async function signResume(
  buffer: Buffer,
  signingKeys: Buffer,
  signingCertPassword: string | undefined
): Promise<Buffer> {
  try {
    const pdfWithPlaceholder: Buffer = plainAddPlaceholder({
      contactInfo: siteConfig.email,
      location: siteConfig.location,
      name: siteConfig.fullName,
      pdfBuffer: buffer,
      reason: 'Digital Resume Verification',
      signatureLength: 16_192,
    })

    // Create P12Signer instance with certificate and passphrase
    const signer: P12Signer = new P12Signer(signingKeys, {
      passphrase: signingCertPassword ?? '',
    })

    // Sign the PDF using the signer instance
    return await signpdf.sign(pdfWithPlaceholder, signer)
  } catch (error) {
    console.error('⚠ Failed to sign PDF:', error)
    return buffer
  }
}

// Main execution wrapped in async IIFE to support CJS output format
void (async (): Promise<void> => {
  const publicDirectory: string = path.join(process.cwd(), 'public', 'resume')
  const signingCertBase64: string | undefined = await getSecret(
    'RESUME_SIGNING_CERT_BASE64'
  )
  const signingCertPassword: string | undefined = await getSecret(
    'RESUME_SIGNING_CERT_PASSWORD'
  )

  // 1. Clean and Create Directory FIRST
  try {
    await rm(publicDirectory, { force: true, recursive: true })
    console.log('✓ Cleaned resume directory')
  } catch {
    // Directory doesn't exist, that's fine
  }
  await mkdir(publicDirectory, { recursive: true })

  // 2. Prepare Signing Keys and Export Certificate
  let signingKeys: Buffer | undefined

  if (signingCertBase64 !== undefined) {
    try {
      signingKeys = Buffer.from(signingCertBase64, 'base64')
      await exportCertificateAssets(
        signingKeys,
        signingCertPassword,
        publicDirectory
      )
    } catch (error) {
      console.error('⚠ Invalid signing certificate or password:', error)
    }
  }

  // 3. Generate Resumes
  for (const locale of routing.locales) {
    console.log(`Generating resume for locale: ${locale}`)

    // Load messages for the locale
    const messages: Messages =
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, unicorn/no-await-expression-member, no-unsanitized/method
      (await import(`../messages/${locale}.json`)).default as Messages

    const translations: Translations<''> = createTranslator({
      locale,
      messages,
    })

    // Explicitly set timezone to UTC (or any valid IANA zone) to prevent system-dependent fail
    const formatter: ReturnType<typeof createFormatter> = createFormatter({
      locale,
      timeZone: 'UTC',
    })

    const element: React.ReactElement = React.createElement(ResumePDFDocument, {
      formatDate: formatter,
      translations,
    })

    // @ts-expect-error - React PDF type mismatch
    const buffer: Buffer = await renderToBuffer(element)

    let finalBuffer: Buffer = buffer

    if (signingKeys !== undefined) {
      finalBuffer = await signResume(buffer, signingKeys, signingCertPassword)
      console.log(`  ✓ Signed ${locale}.pdf`)
    }

    const filename: string = `${locale}.pdf`
    await writeFile(path.join(publicDirectory, filename), finalBuffer)

    console.log(`✓ Generated ${filename}`)
  }

  console.log('Resume generation complete!')
})()
