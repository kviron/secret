import { Component, JSX } from 'solid-js';
import { Sidebar } from '@/widgets/Sidebar';
import './AppLayout.css';

interface AppLayoutProps {
  children?: JSX.Element;
}

export const AppLayout: Component<AppLayoutProps> = (props) => {
  return (
    <div class="app-layout">
      <Sidebar />
      <main class="main-content">
        {props.children}
      </main>
    </div>
  );
};