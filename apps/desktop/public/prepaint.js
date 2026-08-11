(() => {
  const locale = localStorage.getItem("lifeos.locale") === "ar" ? "ar" : "en";
  document.documentElement.lang = locale;
  document.documentElement.dir = locale === "ar" ? "rtl" : "ltr";
})();
