import bech32 from 'bech32';
import { fromBech32 } from '@cosmjs/encoding';
import { encodePubkey } from '@cosmjs/proto-signing';
import { Ed25519Pubkey, encodeSecp256k1Pubkey } from '@cosmjs/amino';
import { Ed25519 } from '@cosmjs/crypto';

// Функция для конвертации Bech32 адреса валидатора в consensus_pubkey
function convertValidatorAddressToConsensusPubkey(bech32Address: string): any {
  // Декодируем Bech32 адрес
  const { data } = fromBech32(bech32Address);
  console.log(data);
  // const pubkey = encodeSecp256k1Pubkey(data);
  // console.log(pubkey);

  // const pkey = Ed25519;

  // Предполагаем, что используется Ed25519 публичный ключ
  const pubkey: Ed25519Pubkey = {
    type: 'tendermint/PubKeyEd25519',
    value: Buffer.from(data).toString('base64'),
  };

  console.log(pubkey);

  // Кодируем публичный ключ в формат, ожидаемый Cosmos SDK
  const encodedPubkey = encodePubkey(pubkey);

  console.log(encodedPubkey);

  console.log(Buffer.from(encodedPubkey.value).toString('base64'));

  return encodedPubkey;
}

// Пример использования
const bech32Address = 'cosmosvalcons1lx7wgz20q0xsjrn6ksflhjmpf80m7phvfdkzwf';
convertValidatorAddressToConsensusPubkey(bech32Address)
  .then((consensusPubkey) => console.log(consensusPubkey))
  .catch((error) => console.error('Ошибка при конвертации:', error));
