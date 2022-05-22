import { css } from '@emotion/react';
import { SpinButton, TextField } from '@fluentui/react';

export default function Settings(): JSX.Element {
  return (
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
      />
      <SpinButton
        css={css`
          justify-content: space-between;
        `}
        label="使用する TCP ポート"
        styles={{
          labelWrapper: {
            height: 'auto',
          },
          spinButtonWrapper: { width: 0 },
          input: { textAlign: 'end', textOverflow: 'clip' },
        }}
        max={65535}
        min={1}
        // value={String(props.settings.peerCastPort)}
        // onChange={(_ev, newValue) =>
        //   props.onChange({
        //     ...props.settings,
        //     peerCastPort: Number(newValue),
        //   })
        // }
        value={''}
      />
    </div>
  );
}
