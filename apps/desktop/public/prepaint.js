(() => {
  const key = "lifeos.presentation.v1";
  let saved = {};
  try {
    saved = JSON.parse(localStorage.getItem(key) || "{}");
  } catch {
    saved = {};
  }
  const locale = saved.locale === "ar" ? "ar" : "en";
  const preference = ["light", "dark", "system"].includes(saved.theme)
    ? saved.theme
    : "system";
  const theme =
    preference === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : preference;
  document.documentElement.lang = locale;
  document.documentElement.dir = locale === "ar" ? "rtl" : "ltr";
  document.documentElement.dataset.theme = theme;
  document.documentElement.dataset.themePreference = preference;
})();
