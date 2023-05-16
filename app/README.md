# create-svelte

## Misc Notes
- after each testnet contract deployment, the following variables need to be updated:
  - /app/src/lib/contracts.ts - update the Liquidity Book Contracts section
  - /app/src/lib/tokens.ts - update token X and Y

## Developing

Once you've created a project and installed dependencies with `npm install` (or `pnpm install` or `yarn`), start a development server:

```bash
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

To create a production version of your app:

```bash
npm run build
```

You can preview the production build with `npm run preview`.

> To deploy your app, you may need to install an [adapter](https://kit.svelte.dev/docs/adapters) for your target environment.
