import { css } from '@emotion/react';
import { IconButton, Toggle } from '@fluentui/react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { useEffect, useState } from 'react';
import ConnectionsView, { Connections } from './components/ConnectionsView';
import SettingsView, { Settings } from './components/SettingsView';
import dummyData from './utils/dummyData';

export type AtomOrRaw = Atom | string;

export type Atom = { children?: readonly Atom[] } & (AtomParent | AtomChild);

export interface AtomParent {
  identifier: string;
  children: readonly Atom[];
}

export interface AtomChild {
  identifier: string;
  payload?: string | readonly number[];
}

function isDataChannelPacket(atom: Atom): boolean {
  return (
    atom.identifier === 'chan' &&
    atom.children?.length === 2 &&
    !!atom.children
      ?.find((x) => x.identifier === 'pkt\0')
      ?.children?.some(
        (x) => x.identifier === 'type' && 'payload' in x && x.payload === 'data'
      )
  );
}

function updatedConnections(
  connections: Connections,
  payload: JsonPayload,
  isSkipDataPacket: boolean
): Connections {
  let atom: Atom;
  if (payload.type === 'atom') {
    atom = payload.payload as Atom;
  } else if (payload.type === 'raw') {
    atom = {
      identifier: '#RAW',
      payload: payload.payload as string,
    };
  } else if (payload.type === 'info') {
    atom = {
      identifier: '#IFO',
      payload: payload.payload as string,
    };
  } else {
    atom = {
      identifier: '#UNK',
      payload: payload.type,
    };
  }
  const key = `${payload.clientHost}_${payload.serverHost}`;
  let connection = connections[key] ?? {
    clientHost: payload.clientHost,
    serverHost: payload.serverHost,
    uploadStream: [],
    downloadStream: [],
  };
  switch (payload.direction) {
    case 'upload':
      connection = {
        ...connection,
        uploadStream: [...connection.uploadStream, atom],
      };
      break;
    case 'download':
      if (
        isSkipDataPacket &&
        connection.downloadStream.length > 0 &&
        isDataChannelPacket(
          connection.downloadStream[connection.downloadStream.length - 1]
        ) &&
        isDataChannelPacket(atom)
      ) {
        break;
      }
      connection = {
        ...connection,
        downloadStream: [...connection.downloadStream, atom],
      };
      break;
    default:
      throw new Error();
  }
  return { ...connections, [key]: connection };
}

export interface JsonPayload {
  type: string;
  clientHost: string;
  serverHost: string;
  direction: string;
  payload: AtomOrRaw | null;
}

export default function App(): JSX.Element {
  const [showSettings, setShowSettings] = useState(false);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [connections, setConnections] = useState<{
    [clientHostServerHost: string]: {
      clientHost: string;
      serverHost: string;
      uploadStream: readonly Atom[];
      downloadStream: readonly Atom[];
    };
  }>({});

  useEffect(() => {
    // dummyData.forEach((x) => {
    //   setConnections((connections) =>
    //     updatedConnections(connections, x, settings?.isSkipDataPacket === true)
    //   );
    // });

    const unlistenFnPromise = listen<JsonPayload>('json', (ev) => {
      setConnections((connections) =>
        updatedConnections(
          connections,
          ev.payload,
          settings?.isSkipDataPacket === true
        )
      );
    });
    (async () => {
      const initialData: any = await invoke('initial_data');
      setSettings(initialData.settings);
    })();

    return () => {
      unlistenFnPromise.then((x) => x());
    };
  }, []);

  return (
    <div
      css={css`
        height: 100%;
        overflow: hidden;
        display: flex;
        flex-direction: column;
      `}
    >
      <div>
        {settings == null ? null : (
          <div
            hidden={!showSettings}
            css={css`
              position: absolute;
              width: 100%;
              z-index: 1;
              background-color: #fdfdfe;
            `}
          >
            <div
              css={css`
                padding: 8px;
                border: 1px outset;
              `}
            >
              <SettingsView
                defaultValues={settings}
                onClose={() => {
                  setShowSettings(false);
                }}
                onSubmit={async (newSettings) => {
                  await invoke('set_settings', { ...newSettings });
                  setSettings(newSettings);
                  setShowSettings(false);
                  await invoke('restart');
                }}
              />
            </div>
          </div>
        )}
        <div
          css={css`
            display: flex;
            align-items: center;
          `}
        >
          <div
            css={css`
              flex: 1;
              display: flex;
              margin: 0 16px;
              justify-content: space-between;
            `}
          >
            <div>PeerCast: {settings?.realServerHost}</div>
            <div>このマシン: {settings?.ipv4AddrFromRealServer}</div>
            <div>公開ポート: {settings?.ipv4Port}</div>
          </div>
          <div
            css={css`
              text-align: end;
              z-index: 2;
            `}
          >
            <IconButton
              iconProps={{ iconName: 'gear' }}
              onClick={() => setShowSettings((value) => !value)}
            />
          </div>
        </div>
        <Toggle
          label="データパケットをスキップする"
          inlineLabel
          disabled={settings == null}
          checked={settings?.isSkipDataPacket}
          onChange={async (ev, checked) => {
            const newSettings = {
              ...settings!,
              isSkipDataPacket: checked === true,
            };
            await invoke('set_settings', newSettings);
            setSettings(newSettings);
          }}
          css={css`
            display: flex;
            justify-content: end;
            margin-right: 8px;
          `}
        />
      </div>
      <div
        css={css`
          margin-top: 16px;
          flex-grow: 1;
          overflow: hidden;
        `}
      >
        <ConnectionsView connections={connections} />
      </div>
    </div>
  );
}
