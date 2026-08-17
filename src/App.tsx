import { useEffect, useState } from 'react'
import { Routes, Route } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { Layout } from './components/Layout'
import { Home } from './pages/Home'
import { MinimalHome } from './pages/MinimalHome'
import { OnboardingWizard } from './pages/OnboardingWizard'
import { Download } from './pages/Download'
import { Account } from './pages/Account'
import { Settings } from './pages/Settings'
import { Help } from './pages/Help'
import { Library } from './pages/Library'
import { Music } from './pages/Music'
import { Mods } from './pages/Mods'
import { ResourcePacks } from './pages/ResourcePacks'
import { InstanceManagement } from './pages/InstanceManagement'
import { Schematics } from './pages/Schematics'
import { Multiplayer } from './pages/Multiplayer'
import AiCrash from './pages/AiCrash'
import WorldMapPreview from './pages/WorldMapPreview'
import { useSettingsStore } from './stores/settingsStore'
import { useMusicStore } from './stores/musicStore'
import { applyWindowSize } from './utils/windowSize'
import { MaterialProvider } from './components/material'
import type { LauncherConfig } from './types'

function HomeRouter() {
  const { config } = useSettingsStore()
  if (config.home_style === 'minimal') {
    return <MinimalHome />
  }
  return <Home />
}

function App() {
  const { config, setConfig } = useSettingsStore()
  const [loaded, setLoaded] = useState(false)

  useEffect(() => {
    invoke<LauncherConfig>('load_config').then((cfg) => {
      setConfig(cfg)
      applyWindowSize(cfg).catch(console.error)
      setLoaded(true)
    }).catch(() => setLoaded(true))
  }, [])

  useEffect(() => {
    void useMusicStore.getState().reloadPlaylist()
  }, [])

  const needsOnboarding = loaded && !config.onboarding_completed

  const handleOnboardingComplete = () => {
    invoke<LauncherConfig>('load_config').then((cfg) => {
      setConfig(cfg)
    }).catch(() => {})
  }

  if (!loaded) {
    return (
      <MaterialProvider>
        <Routes>
          <Route element={<Layout />}>
            <Route path="*" element={<Home />} />
          </Route>
        </Routes>
      </MaterialProvider>
    )
  }

  return (
    <MaterialProvider>
      {needsOnboarding ? (
        <OnboardingWizard onComplete={handleOnboardingComplete} />
      ) : (
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<HomeRouter />} />
            <Route path="/download" element={<Download />} />
            <Route path="/account" element={<Account />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/help" element={<Help />} />
            <Route path="/library" element={<Library />} />
            <Route path="/music" element={<Music />} />
            <Route path="/multiplayer" element={<Multiplayer />} />
            <Route path="/ai" element={<AiCrash />} />
            <Route path="/worlds/:instanceId" element={<WorldMapPreview />} />
            <Route path="/instances/:instanceId/mods" element={<Mods />} />
            <Route path="/instances/:instanceId/resourcepacks" element={<ResourcePacks />} />
            <Route path="/instances/:instanceId/schematics" element={<Schematics />} />
            <Route path="/instances/:instanceId/manage" element={<InstanceManagement />} />
          </Route>
        </Routes>
      )}
    </MaterialProvider>
  )
}

export default App
