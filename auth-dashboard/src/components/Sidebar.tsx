import React from 'react';
import './Sidebar.css';
import { SIDEBAR_MENU_ORDER, APP_SHORT_NAME, APP_VERSION } from '../constants/app.constants';

interface SubMenu {
  id: string;
  label: string;
  icon: string;
}

interface SidebarProps {
  activeMenu: string;
  onMenuChange: (menuId: string) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ activeMenu, onMenuChange }) => {
  // 부모 메뉴가 활성화되었는지 확인 (하위 메뉴 포함)
  const isMenuActive = (menuId: string, subMenus?: readonly SubMenu[]) => {
    if (activeMenu === menuId) return true;
    if (subMenus) {
      return subMenus.some(sub => activeMenu === sub.id);
    }
    return false;
  };

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <h2 className="sidebar-title">{APP_SHORT_NAME}</h2>
        <span className="sidebar-version">v{APP_VERSION}</span>
      </div>

      <nav className="sidebar-nav">
        {SIDEBAR_MENU_ORDER.map((menu) => {
          const hasSubMenus = 'subMenus' in menu && menu.subMenus;
          const isActive = isMenuActive(menu.id, hasSubMenus ? menu.subMenus : undefined);

          return (
            <div key={menu.id} className="sidebar-menu-group">
              <button
                className={`sidebar-menu-item ${isActive ? 'active' : ''}`}
                onClick={() => {
                  // 하위 메뉴가 있으면 첫 번째 하위 메뉴로, 없으면 메인 메뉴로
                  if (hasSubMenus && menu.subMenus.length > 0) {
                    onMenuChange(menu.subMenus[0].id);
                  } else {
                    onMenuChange(menu.id);
                  }
                }}
              >
                <span className="menu-icon">{menu.icon}</span>
                <div className="menu-content">
                  <span className="menu-label">{menu.label}</span>
                  <span className="menu-description">{menu.description}</span>
                </div>
                {hasSubMenus && (
                  <span className="menu-arrow">{isActive ? '▼' : '▶'}</span>
                )}
              </button>

              {/* 하위 메뉴 렌더링 */}
              {hasSubMenus && isActive && (
                <div className="sidebar-submenu">
                  {menu.subMenus.map((sub) => (
                    <button
                      key={sub.id}
                      className={`sidebar-submenu-item ${activeMenu === sub.id ? 'active' : ''}`}
                      onClick={() => onMenuChange(sub.id)}
                    >
                      <span className="submenu-icon">{sub.icon}</span>
                      <span className="submenu-label">{sub.label}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          );
        })}
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

