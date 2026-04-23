# jarvis-mobile

Expo (React Native) client for Jarvis: **relay terminal** (encrypted PTY bridge to desktop), **embedded livechat** (Supabase + Web Crypto in a WebView), and **Claude Code** (claude.ai in a WebView).

Parent repo: [github.com/dyoburon/jarvis](https://github.com/dyoburon/jarvis) — see root [README.md](../README.md) and [ARCHITECTURE.md](../ARCHITECTURE.md).

## Run

```bash
cd jarvis-mobile
npm install
npx expo start
```

Then open the iOS simulator, Android emulator, or Expo Go / dev client as usual.

## Scripts

| Script | Purpose |
|--------|---------|
| `npm run lint` | ESLint ([.eslintrc.cjs](.eslintrc.cjs) + `eslint-config-expo`) |
| `npm run typecheck` | `tsc --noEmit` |
| `npm run test` | Jest (`jest-expo`) — pairing/linking, mock relay, fake WebSocket relay handshake, `buildChatHTML` Supabase injection, ECDH alignment with Rust `derive_shared_key` |
| `npm run sync:chat-html` | Copy `jarvis-rs/assets/panels/chat/index.html` → `vendor/chat-index.from-jarvis-rs.html` for diff/review |

## Tabs

- **[ code ]** — Paste or scan pairing data, connect to the relay, drive a remote terminal. Session token is stored locally (AsyncStorage).
- **[ chat ]** — Livechat UI inlined in [lib/jarvis-chat-html.ts](lib/jarvis-chat-html.ts); Supabase URL and anon key are injected at build time from `EXPO_PUBLIC_SUPABASE_*` (defaults match the bundled project if unset). HTTP errors show an offline banner (distinct from relay issues).
- **[ claude ]** — `https://claude.ai/code` in a WebView. Google / Microsoft / Apple OAuth navigations are opened with `expo-web-browser` **auth session** + `jarvis://oauth/callback` when possible; use **[browser]** or **[reload]** if sign-in still fails. Cookies are not automatically shared back into the WebView.

## Deep links

Scheme: `jarvis` (see [app.json](app.json)). Opening **`jarvis://pair?relay=...&session=...&dhpub=...`** queues pairing and navigates home so the code tab can connect once the terminal WebView is ready.

## Environment (optional)

See [.env.example](.env.example). Public build-time variables (no secrets):

- `EXPO_PUBLIC_DEFAULT_RELAY_URL` — shown as a hint in Settings when set.
- `EXPO_PUBLIC_SUPABASE_URL` / `EXPO_PUBLIC_SUPABASE_ANON_KEY` — override embedded livechat Supabase client (anon key is public by design).
- `EXPO_PUBLIC_RELAY_DEBUG=1` — on the code tab, show the last relay protocol message type (no payloads).

## Help & UX

- Modal **[ help ]** ([app/help.tsx](app/help.tsx)) from Settings or **[help]** on the code tab — explains relay vs chat vs Claude and offline behavior.
- Tab navigator uses `freezeOnBlur: false` and capped **font scaling** ([lib/theme.ts](lib/theme.ts)) so WebViews and the terminal row stay usable with large accessibility text.

## EAS Build

[eas.json](eas.json) defines `development`, `preview`, and `production` profiles. Install [EAS CLI](https://docs.expo.dev/build/setup/) and run `eas build` when you are ready to ship.

## Threat model (short)

- **Relay:** After both sides join the relay session, mobile and desktop perform ECDH + AES-GCM for PTY messages (see [lib/crypto.ts](lib/crypto.ts), [lib/relay-connection.ts](lib/relay-connection.ts)).
- **Livechat:** E2E crypto runs in the chat WebView (Web Crypto), separate from the relay path.
- Do not share pairing strings with untrusted apps or screenshots.

## Chat HTML parity

Desktop chat uses `window.jarvis.ipc` for crypto; mobile uses Web Crypto. The sync script does **not** auto-merge those layers — use `vendor/chat-index.from-jarvis-rs.html` to see upstream changes and update [lib/jarvis-chat-html.ts](lib/jarvis-chat-html.ts) manually.
