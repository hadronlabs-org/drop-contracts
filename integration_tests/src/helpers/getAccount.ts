import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';

export const getAccount = async (
  mnemonic: string,
  prefix: string,
): Promise<AccountData> => {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix,
  });
  const accounts = await wallet.getAccounts();
  return accounts[0];
};
