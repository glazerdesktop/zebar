import './index.css';
import { render } from 'solid-js/web';
import { createStore } from 'solid-js/store';
import * as zebar from 'zebar';
import { createSignal, createEffect } from 'solid-js';

const providers = zebar.createProviderGroup({
  audio: { type: 'audio' },
  cpu: { type: 'cpu' },
  focusedWindow: { type: 'focusedWindow' },
  battery: { type: 'battery' },
  memory: { type: 'memory' },
  weather: { type: 'weather' },
  media: { type: 'media' },
});

render(() => <App />, document.getElementById('root')!);

function App() {
  const [output, setOutput] = createStore(providers.outputMap);
  const [iconUrl, setIconUrl] = createSignal<string | null>(null);
  const [lastTitle, setLastTitle] = createSignal<string | null>(null);

  providers.onOutput(outputMap => {
    setOutput(outputMap);

    // Only process icon if window title has changed
    if (outputMap.focusedWindow?.title !== lastTitle()) {
      setLastTitle(outputMap.focusedWindow?.title || null);

      const icon = outputMap.focusedWindow?.icon;
      if (icon) {
        try {
          const binary = atob(icon);
          const array = new Uint8Array(binary.length);
          for (let i = 0; i < binary.length; i++) {
            array[i] = binary.charCodeAt(i);
          }
          const blob = new Blob([array], { type: 'image/png' });

          // Clean up old URL if it exists
          if (iconUrl()) {
            URL.revokeObjectURL(iconUrl()!);
          }

          const url = URL.createObjectURL(blob);
          setIconUrl(url);
        } catch (error) {
          console.error('Error decoding icon data:', error);
          setIconUrl(null);
        }
      } else {
        setIconUrl(null);
      }
    }
  });

  return (
    <div class="app">
      <div class="chip">
        {output.audio?.defaultPlaybackDevice?.name} -
        {output.audio?.defaultPlaybackDevice?.volume}
      </div>
      <div class="chip">
        Media: {output.media?.currentSession?.title} -
        {output.media?.currentSession?.artist}
        <button onClick={() => output.media?.togglePlayPause()}>⏯</button>
      </div>
      <div class="chip">CPU usage: {output.cpu?.usage}</div>
      {iconUrl() && (
        <img height="20" width="20" src={iconUrl()!} alt="icon" />
      )}
      <div class="chip">Focused window: {output.focusedWindow?.title}</div>
      <div class="chip">
        Battery charge: {output.battery?.chargePercent}
      </div>
      <div class="chip">Memory usage: {output.memory?.usage}</div>
      <div class="chip">Weather temp: {output.weather?.celsiusTemp}</div>
    </div>
  );
}
