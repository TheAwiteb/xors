import "./nav.css";
import { Link } from "react-router-dom";
import { useState } from "react";
import { NavLink } from "react-router-dom";

const Nav = () => {
  const img = null;
  const [menu, setMenu] = useState(false);
  const [mode, setMode] = useState(
    window.matchMedia("(prefers-color-scheme: light)").matches ||
      localStorage.getItem("mode")
  );
  setThemePreference();

  function changeMode() {
    setMode(!mode);
  }

  function setThemePreference() {
    if (mode == true) enableLightMode();
    if (mode == false) enableDarkMode();
  }

  function enableDarkMode() {
    document.body.classList.remove("light-theme");
    document.body.classList.add("dark-theme");
    localStorage.setItem("mode", "0");
  }

  function enableLightMode() {
    document.body.classList.remove("dark-theme");
    document.body.classList.add("light-theme");
    localStorage.setItem("mode", "1");
  }

  return (
    <nav>
      <div className="container">
        <Link className="logo" to="/">
          O X
        </Link>
        <div className={menu ? "links active" : "links"}>
          <NavLink onClick={() => setMenu(false)} to="/">
            الرئيسية
          </NavLink>
          <NavLink onClick={() => setMenu(false)} to="choose">
            العب
          </NavLink>
        </div>

        <div className="account">
          <div className="mode">
            <button
              onClick={() => changeMode()}
              id="theme-toggle"
              aria-label="تبديل طريقة العرض"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 472.39 472.39"
              >
                <g className="toggle-sun">
                  <path d="M403.21,167V69.18H305.38L236.2,0,167,69.18H69.18V167L0,236.2l69.18,69.18v97.83H167l69.18,69.18,69.18-69.18h97.83V305.38l69.18-69.18Zm-167,198.17a129,129,0,1,1,129-129A129,129,0,0,1,236.2,365.19Z" />
                </g>
                <g className="toggle-circle">
                  <circle className="cls-1" cx="236.2" cy="236.2" r="103.78" />
                </g>
              </svg>
            </button>
          </div>
          <Link className="user" to="profile">
            <img src={img ?? "assets/user.svg"} alt="الحساب" />
          </Link>
          <span
            className={menu ? "menu-icon close" : "menu-icon"}
            onClick={() => {
              setMenu((prev) => !prev);
            }}
          >
            <svg viewBox="0 0 32 32">
              <path
                className="line line-top-bottom"
                d="M27 10 13 10C10.8 10 9 8.2 9 6 9 3.5 10.8 2 13 2 15.2 2 17 3.8 17 6L17 26C17 28.2 18.8 30 21 30 23.2 30 25 28.2 25 26 25 23.8 23.2 22 21 22L7 22"
              ></path>
              <path className="line" d="M7 16 27 16"></path>
            </svg>
          </span>
        </div>
      </div>
    </nav>
  );
};

export default Nav;
