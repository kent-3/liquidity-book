import {
  subscribeEvent,
  type EventCallback,
  unsubscribeEventCallback,
  type CleanListenerCallback,
} from './index';
import { getWindow, accountChangedCallback, provider } from '..';

/**
 * `account` event gets triggered when the Account Provider has an account ready.
 */
export function onAccountAvailable(
  callback: EventCallback
): CleanListenerCallback {
  subscribeEvent('account-available', callback);
  return unsubscribeEventCallback('account-available', callback);
}

/**
 * `account-change` events gets triggered when the account change on the Account
 * Provider.
 */
export function onAccountChange(
  callback: EventCallback
): CleanListenerCallback {
  subscribeEvent('account-change', callback);
  // TODO: Fix calling API directly
  return () => {
    if (!provider) {
      throw new Error('No provider available');
    }
    getWindow()?.removeEventListener('account-change', callback);
    if (!accountChangedCallback) return;
    getWindow()?.removeEventListener(
      'keplr_keystorechange',
      accountChangedCallback
    );
  };
}

export function onAccountNotAvailable(
  callback: EventCallback
): CleanListenerCallback {
  subscribeEvent('account-not-available', callback);
  return unsubscribeEventCallback('account-not-available', callback);
}

export function onAccountDisconnect(
  callback: EventCallback
): CleanListenerCallback {
  subscribeEvent('shutdown', callback);
  return unsubscribeEventCallback('shutdown', callback);
}
