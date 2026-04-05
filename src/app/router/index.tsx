import { Route, Router } from '@solidjs/router';
import { AppLayout } from '@/widgets/AppLayout';
import { DashboardPage } from '@/pages/dashboard';
import { GameScopeLayout } from '@/pages/game-scope/GameScopeLayout';
import { GameIndexRedirect } from '@/pages/game-scope/GameIndexRedirect';
import { GameModsPage } from '@/pages/game-mods';
import { GamePluginsPage } from '@/pages/game-plugins';
import { GameSavesPage } from '@/pages/game-saves';
import { ModsPage } from '@/pages/mods';
import { DeploymentsPage } from '@/pages/deployments';
import { SettingsPage } from '@/pages/settings';

export const AppRouter = () => {
  return (
    <Router root={AppLayout}>
      <Route path="/" component={DashboardPage} />
      <Route path="/game/:id" component={GameScopeLayout}>
        <Route path="/" component={GameIndexRedirect} />
        <Route path="mods" component={GameModsPage} />
        <Route path="plugins" component={GamePluginsPage} />
        <Route path="saves" component={GameSavesPage} />
      </Route>
      <Route path="/mods" component={ModsPage} />
      <Route path="/deployments" component={DeploymentsPage} />
      <Route path="/settings" component={SettingsPage} />
    </Router>
  );
};
