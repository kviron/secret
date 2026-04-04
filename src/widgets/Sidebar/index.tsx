import { Component, createSignal, For, Show } from 'solid-js';
import { A, useLocation } from '@solidjs/router';
import './Sidebar.css';

interface NavItem {
  path: string;
  label: string;
  icon: string;
}

const navItems: NavItem[] = [
  { path: '/', label: 'Games', icon: '🎮' },
  { path: '/mods', label: 'Mods', icon: '📦' },
  { path: '/deployments', label: 'Deployments', icon: '🚀' },
  { path: '/settings', label: 'Settings', icon: '⚙️' },
];

export const Sidebar: Component = () => {
  const [collapsed, setCollapsed] = createSignal(false);
  const location = useLocation();

  const isActive = (path: string) => {
    if (path === '/') return location.pathname === '/';
    return location.pathname.startsWith(path);
  };

  return (
    <aside class="sidebar" classList={{ 'sidebar-collapsed': collapsed() }}>
      <div class="sidebar-header">
        <div class="sidebar-brand">
          <span class="brand-icon">⚔️</span>
          <Show when={!collapsed()}>
            <span class="brand-text">Pantheon</span>
          </Show>
        </div>
        <button
          class="sidebar-toggle"
          onClick={() => setCollapsed(!collapsed())}
          title={collapsed() ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          <span class="toggle-icon">{collapsed() ? '→' : '←'}</span>
        </button>
      </div>

      <nav class="sidebar-nav">
        <For each={navItems}>
          {(item) => (
            <A
              href={item.path}
              class="nav-item"
              classList={{ 'nav-active': isActive(item.path) }}
              title={collapsed() ? item.label : undefined}
            >
              <span class="nav-icon">{item.icon}</span>
              <Show when={!collapsed()}>
                <span class="nav-label">{item.label}</span>
              </Show>
            </A>
          )}
        </For>
      </nav>
    </aside>
  );
};