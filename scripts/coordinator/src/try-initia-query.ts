import { fromHex, toBech32 } from '@cosmjs/encoding';
import { LCDClient, MnemonicKey, Wallet } from '@initia/initia.js';

async function main() {
  const lcd = new LCDClient('https://rest-kyrthos.interwoven-1.ntrn.tech:443', {
    chainId: 'interwoven-1',
    gasPrices: '0.15uinit',
    gasAdjustment: '2',
  });
  const key = new MnemonicKey({
    mnemonic:
      'object stove check bronze enable audit build craft heavy mobile fantasy robust pond talent box animal drip spell current kidney truck dragon pig illness',
  });
  const wallet = new Wallet(lcd, key);

  const lpModuleObject =
    '0x2ecfb78e44ae0b898d8ffbae7c5c0ff9a776ca61dd3514353c6339321d08dac4';

  const bech32Address = toBech32('init', fromHex(lpModuleObject.substring(2)));
  console.log(`bech32Address: ${bech32Address}`);

  const coin = await lcd.bank.balanceByDenom(bech32Address, 'uinit');
  console.log(`coin: ${JSON.stringify(coin)}`);
}

main();
