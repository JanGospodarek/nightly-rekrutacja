import { mount, StartClient } from 'solid-start/entry-client'
import { Buffer } from 'buffer'
window.Buffer = Buffer

mount(() => <StartClient />, document)
