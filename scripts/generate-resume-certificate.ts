/* eslint-disable no-console */
/* eslint-disable @typescript-eslint/typedef */
/* eslint-disable unicorn/prefer-top-level-await */
/* eslint-disable no-secrets/no-secrets */
import crypto from 'node:crypto'
import { writeFile } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'

import forge from 'node-forge'

import { siteConfig } from '@/lib/config'

const CERT_FILENAME: string = 'resume-signing-cert.p12'

// Get password from argument or generate a secure one
const passwordArgument: string | undefined = process.argv[2]
const CERT_PASSWORD: string =
  passwordArgument ?? crypto.randomBytes(32).toString('hex')

void (async (): Promise<void> => {
  // ... (rest of file)
  console.log('Generating self-signed certificate...')
  console.log('This might take a moment due to 4096-bit key generation...')

  // Generate a keypair
  const keys = forge.pki.rsa.generateKeyPair({ bits: 4096, workers: -1 })

  // Create a certificate
  const cert = forge.pki.createCertificate()
  cert.publicKey = keys.publicKey
  cert.serialNumber = '01'
  cert.validity.notBefore = new Date()
  cert.validity.notAfter = new Date()
  cert.validity.notAfter.setFullYear(cert.validity.notBefore.getFullYear() + 2) // 2 years validity

  const attributes = [
    { name: 'commonName', value: `${siteConfig.fullName} Resume Signer` },
    { name: 'countryName', value: 'DE' },
    { shortName: 'ST', value: 'DE' },
    { name: 'localityName', value: 'DE' },
    { name: 'organizationName', value: siteConfig.fullName },
    { shortName: 'OU', value: siteConfig.jobTitle },
    { name: 'emailAddress', value: siteConfig.email },
  ]
  cert.setSubject(attributes)
  cert.setIssuer(attributes)

  // Extensions
  cert.setExtensions([
    {
      cA: false,
      name: 'basicConstraints',
    },
    {
      dataEncipherment: true,
      digitalSignature: true,
      keyCertSign: false,
      keyEncipherment: true,
      name: 'keyUsage',
      nonRepudiation: true,
    },
    {
      clientAuth: false,
      codeSigning: false,
      emailProtection: true,
      name: 'extKeyUsage',
      serverAuth: false,
      timeStamping: true,
    },
    {
      client: true,
      email: true,
      emailCA: false,
      name: 'nsCertType',
      objCA: false,
      objsign: false,
      server: false,
      sslCA: false,
    },
    {
      altNames: [
        {
          type: 1, // RFC 822 email
          value: siteConfig.email,
        },
        {
          type: 6, // URI
          value: siteConfig.url,
        },
      ],
      name: 'subjectAltName',
    },
    {
      name: 'subjectKeyIdentifier',
    },
  ])

  // Sign the certificate
  cert.sign(keys.privateKey, forge.md.sha256.create())

  // Create P12
  const p12Asn1 = forge.pkcs12.toPkcs12Asn1(
    keys.privateKey,
    [cert],
    CERT_PASSWORD,
    { algorithm: '3des' }
  )

  const p12Der = forge.asn1.toDer(p12Asn1).getBytes()
  const p12Buffer = Buffer.from(p12Der, 'binary')

  const outputPath = path.join(process.cwd(), CERT_FILENAME)
  await writeFile(outputPath, p12Buffer)

  console.log(`\n✓ Generated ${CERT_FILENAME}`)
  console.log(`\nTo use this certificate locally:`)
  console.log(`1. Encode it to base64:`)
  if (os.platform() === 'win32') {
    const base64: string = p12Buffer.toString('base64')
    console.log(`   $env:RESUME_SIGNING_CERT_BASE64 = "${base64}"`)
    console.log(`   $env:RESUME_SIGNING_CERT_PASSWORD = "${CERT_PASSWORD}"`)
  } else {
    // For Linux/Mac, print the export commands with explicit values
    const base64: string = p12Buffer.toString('base64')
    console.log(`   export RESUME_SIGNING_CERT_BASE64="${base64}"`)
    console.log(`   export RESUME_SIGNING_CERT_PASSWORD="${CERT_PASSWORD}"`)
  }

  console.log(`\n2. Run the resume generation script:`)
  console.log(`   pnpm build:resume`)

  if (passwordArgument === undefined) {
    console.log('\n⚠ NOTE: A secure random password was generated for you.')
    console.log(`Password: ${CERT_PASSWORD}`)
    console.log(
      'You can provide a custom password by running: npx tsx scripts/generate-resume-certificate.ts <password>'
    )
  }
})()
