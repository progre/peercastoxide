import { css } from '@emotion/react';
import { DefaultButton, SpinButton, TextField } from '@fluentui/react';
import { useState } from 'react';

export interface Settings {
  realServerHost: string;
  ipAddrFromRealServer: string;
  listenPort: number;
  isSkipDataPacket: boolean;
}

export default function SettingsView(props: {
  defaultValues: Settings;
  onClose(): void;
  onSubmit(value: Settings): void;
}): JSX.Element {
  const [currentValues, setCurrentValues] = useState(props.defaultValues);
  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
        gap: 24px;
      `}
    >
      <div
        css={css`
          display: flex;
          gap: 8px;

          > div {
            display: flex;
            flex: 1;
            flex-direction: column;
          }
          > div > div > label {
            padding-top: 0;
          }
        `}
      >
        <TextField
          css={css`
            > div {
              height: 100%;
              display: flex;
              flex-direction: column;
              justify-content: space-between;
            }
          `}
          label="PeerCast のアドレスと TCP ポート番号"
          placeholder="localhost:7144"
          value={currentValues.realServerHost}
          onChange={(_e, value) =>
            setCurrentValues((values) => ({
              ...values,
              realServerHost: value ?? '',
            }))
          }
        />
        <TextField
          css={css`
            > div {
              height: 100%;
              display: flex;
              flex-direction: column;
              justify-content: space-between;
            }
          `}
          label="PeerCast から見たこのマシンの IPv4 アドレス"
          placeholder="127.0.0.1"
          value={currentValues.ipAddrFromRealServer}
          onChange={(_e, value) =>
            setCurrentValues((values) => ({
              ...values,
              ipAddrFromRealServer: value ?? '',
            }))
          }
        />
        <SpinButton
          css={css`
            justify-content: space-between;
          `}
          label="公開する TCP ポート番号"
          styles={{
            labelWrapper: {
              height: 'auto',
            },
            spinButtonWrapper: { width: 0 },
            input: { textAlign: 'end', textOverflow: 'clip' },
          }}
          max={65535}
          min={1}
          value={String(currentValues.listenPort)}
          onChange={(_e, value) =>
            setCurrentValues((values) => ({
              ...values,
              listenPort: Number(value ?? ''),
            }))
          }
        />
      </div>
      <div
        css={css`
          display: flex;
          gap: 8px;
          justify-content: end;
        `}
      >
        <DefaultButton
          onClick={() => {
            setCurrentValues(props.defaultValues);
            props.onClose();
          }}
        >
          キャンセル
        </DefaultButton>
        <DefaultButton
          onClick={() => {
            props.onSubmit(currentValues);
          }}
        >
          適用して再起動
        </DefaultButton>
      </div>
    </div>
  );
}
