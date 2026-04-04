import { Route, Router } from '@solidjs/router';
import { AppLayout } from '@/widgets/AppLayout';
import { DashboardPage } from '@/pages/dashboard';
import { GameDetailPage } from '@/pages/game-detail';
import { ModsPage } from '@/pages/mods';
import { DeploymentsPage } from '@/pages/deployments';
import { SettingsPage } from '@/pages/settings';

export const AppRouter = () => {
  return (
    <Router root={AppLayout}>
      <Route path="/" component={DashboardPage} />
      <Route path="/game/:id" component={GameDetailPage} />
      <Route path="/mods" component={ModsPage} />
      <Route path="/deployments" component={DeploymentsPage} />
      <Route path="/settings" component={SettingsPage} />
    </Router>
  );
};