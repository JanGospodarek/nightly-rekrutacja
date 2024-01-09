export enum QueryNetwork {
  SOLANA = 'SOLANA',
  SUI = 'SUI',
  POLKADOT = 'POLKADOT'
}

export interface WalletSelectorItem {
  name: string
  icon: string
  link: string
  detected?: boolean
  recent?: boolean
}

export type FooterLink = {
  description: string
  linkName: string
  linkUrl: string
}

export type FooterData = FooterLink[]

export enum SelectorView {
  DESKTOP_MAIN,
  MOBILE_MAIN,
  MOBILE_QR,
  MOBILE_ALL,
  CONNECTING
}
