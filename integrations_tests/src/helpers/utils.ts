import { MsgGrant } from 'cosmjs-types/cosmos/authz/v1beta1/tx';
import { GenericAuthorization } from 'cosmjs-types/cosmos/authz/v1beta1/authz';
import Long from 'long';
import {
  DeliverTxResponse,
  SigningCosmWasmClient,
} from '@cosmjs/cosmwasm-stargate';
import { AccountData } from '@cosmjs/proto-signing';

export const grantAuthzPermission = async (
  msgTypeUrl: string,
  gaiaClient: SigningCosmWasmClient,
  gaiaAccount: AccountData,
  icaAddress: string,
): Promise<DeliverTxResponse> => {
  const expiration = new Date();
  expiration.setDate(expiration.getDate() + 1);

  const genericAuthorization = GenericAuthorization.fromPartial({
    msg: msgTypeUrl,
  });

  const msgGrant = {
    typeUrl: '/cosmos.authz.v1beta1.MsgGrant',
    value: MsgGrant.fromPartial({
      granter: gaiaAccount.address,
      grantee: icaAddress,
      grant: {
        authorization: {
          typeUrl: '/cosmos.authz.v1beta1.GenericAuthorization',
          value: GenericAuthorization.encode(genericAuthorization).finish(),
        },
        expiration: {
          seconds: Long.fromNumber(expiration.getTime() / 1000),
          nanos: 0,
        },
      },
    }),
  };

  const fee = {
    amount: [
      {
        amount: '2000',
        denom: 'stake',
      },
    ],
    gas: '200000',
  };

  const result = await gaiaClient.signAndBroadcast(
    gaiaAccount.address,
    [msgGrant],
    fee,
  );
  if (result.code !== 0) {
    throw new Error(`Transaction send error: ${result.rawLog}`);
  }

  return result;
};
