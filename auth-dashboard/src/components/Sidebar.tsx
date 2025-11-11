import React from 'react';
import './Sidebar.css';
import { SIDEBAR_MENU_ORDER, APP_SHORT_NAME, APP_VERSION } from '../constants/app.constants';

interface SidebarProps {
  activeMenu: string;
  onMenuChange: (menuId: string) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ activeMenu, onMenuChange }) => {
  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <h2 className="sidebar-title">{APP_SHORT_NAME}</h2>
        <span className="sidebar-version">v{APP_VERSION}</span>
      </div>

      <nav className="sidebar-nav">
        {SIDEBAR_MENU_ORDER.map((menu) => (
          <button
            key={menu.id}
            className={`sidebar-menu-item ${activeMenu === menu.id ? 'active' : ''}`}
            onClick={() => onMenuChange(menu.id)}
          >
            <span className="menu-icon">{menu.icon}</span>
            <div className="menu-content">
              <span className="menu-label">{menu.label}</span>
              <span className="menu-description">{menu.description}</span>
            </div>
          </button>
        ))}
      </nav>

      <div className="sidebar-footer">
        <div className="sidebar-info">
          <p className="info-text">© 2025 PACS</p>
          <p className="info-text">관리 시스템</p>
        </div>
      </div>
    </div>
  );
};

export default Sidebar;

