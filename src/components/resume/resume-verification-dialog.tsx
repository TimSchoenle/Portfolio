'use client'

import React, { type JSX, useState } from 'react'

import { useTranslations } from 'next-intl'

import { Check, Copy, Download, ShieldCheck } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import type { FCWithRequiredChildren } from '@/types/fc'

interface ResumeVerificationDialogProperties {
  readonly children: React.ReactNode
  readonly fingerprint: string
}

interface FingerprintInputProperties {
  readonly copied: boolean
  readonly fingerprint: string
  readonly onCopy: () => void
}

const FingerprintInput: React.FC<FingerprintInputProperties> = ({
  copied,
  fingerprint,
  onCopy,
}: FingerprintInputProperties): JSX.Element => {
  const translate: (key: string) => string = useTranslations(
    'contact.verification'
  )

  return (
    <div className="grid gap-2">
      <Label htmlFor="fingerprint">{translate('fingerprintLabel')}</Label>
      <div className="flex gap-2">
        <Input
          className="font-mono text-xs"
          id="fingerprint"
          readOnly={true}
          value={fingerprint}
        />
        <Button
          aria-label={translate('copyFingerprint')}
          className="shrink-0"
          size="icon"
          variant="outline"
          onClick={onCopy}
        >
          {copied ? (
            <Check className="h-4 w-4 text-green-500" />
          ) : (
            <Copy className="h-4 w-4" />
          )}
        </Button>
      </div>
    </div>
  )
}

export const ResumeVerificationDialog: FCWithRequiredChildren<
  ResumeVerificationDialogProperties
> = ({
  children,
  fingerprint,
}: ResumeVerificationDialogProperties): JSX.Element => {
  const translate: (key: string) => string = useTranslations(
    'contact.verification'
  )
  const [copied, setCopied]: [
    boolean,
    React.Dispatch<React.SetStateAction<boolean>>,
  ] = useState<boolean>(false)

  const handleCopy: () => void = (): void => {
    void navigator.clipboard.writeText(fingerprint).then((): void => {
      setCopied(true)
      setTimeout((): void => {
        setCopied(false)
      }, 2000)
    })
  }

  return (
    <Dialog>
      <DialogTrigger asChild={true}>{children}</DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <ShieldCheck className="h-5 w-5 text-brand" />
            {translate('title')}
          </DialogTitle>
          <DialogDescription>{translate('description')}</DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <FingerprintInput
            copied={copied}
            fingerprint={fingerprint}
            onCopy={handleCopy}
          />

          <div className="rounded-md border border-yellow-500/20 bg-yellow-500/10 p-4 text-sm text-yellow-500">
            {translate('selfSignedWarning')}
          </div>
        </div>

        <div className="flex justify-end gap-2">
          <Button asChild={true} variant="secondary">
            <a download="certificate.crt" href="/resume/certificate.crt">
              <Download className="mr-2 h-4 w-4" />
              {translate('downloadCertificate')}
            </a>
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
