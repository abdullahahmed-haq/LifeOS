import { useEffect, useState } from "react";
import {
  Link,
  Outlet,
  RouterProvider,
  createRootRoute,
  createRoute,
  createRouter,
} from "@tanstack/react-router";
import {
  Button,
  Dialog,
  DialogTrigger,
  Heading,
  Modal,
  ModalOverlay,
} from "react-aria-components";
import { useIntl } from "react-intl";
import { AreaScreen } from "../features/areas/area-screen";
import { ArchiveScreen } from "../features/lifecycle/archive-screen";
import { TrashScreen } from "../features/lifecycle/trash-screen";
import { VersionHistoryScreen } from "../features/lifecycle/version-history-screen";
import { GeneralSettingsScreen } from "../features/settings/general-settings-screen";
import { usePresentation } from "./presentation";

const rootRoute = createRootRoute({ component: ApplicationShell });
const areasRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: AreaScreen,
});
const homeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "home",
  component: RoutePlaceholder,
  staticData: { messageId: "route.home" },
});
const todayRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "today",
  component: RoutePlaceholder,
});
const projectsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "projects",
  component: RoutePlaceholder,
});
const trashRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "trash",
  component: TrashScreen,
});
const archiveRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "archive",
  component: ArchiveScreen,
});
const versionHistoryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "versions/$entityId",
  component: VersionHistoryScreen,
});
const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "settings/general",
  component: GeneralSettingsScreen,
});
const integrationsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "integrations/$section",
  component: RoutePlaceholder,
});
const sectionRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "$section",
  component: RoutePlaceholder,
});

const routeTree = rootRoute.addChildren([
  areasRoute,
  homeRoute,
  todayRoute,
  projectsRoute,
  archiveRoute,
  versionHistoryRoute,
  trashRoute,
  settingsRoute,
  integrationsRoute,
  sectionRoute,
]);
const router = createRouter({ routeTree, defaultPreload: "intent" });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

export function LifeOSRouter() {
  return <RouterProvider router={router} />;
}

function ApplicationShell() {
  const intl = useIntl();
  const { locale, setLocale, theme, setTheme } = usePresentation();
  const [dialog, setDialog] = useState<string | null>(null);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setDialog("command");
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <p className="brand">{intl.formatMessage({ id: "app.name" })}</p>
        <nav aria-label={intl.formatMessage({ id: "navigation.primary" })}>
          <Link to="/home" activeProps={{ "aria-current": "page" }}>
            {intl.formatMessage({ id: "navigation.home" })}
          </Link>
          <Link to="/" activeProps={{ "aria-current": "page" }}>
            {intl.formatMessage({ id: "navigation.areas" })}
          </Link>
          <Link to="/today" activeProps={{ "aria-current": "page" }}>
            {intl.formatMessage({ id: "navigation.today" })}
          </Link>
          <Link to="/projects" activeProps={{ "aria-current": "page" }}>
            {intl.formatMessage({ id: "navigation.projects" })}
          </Link>
          <Link to="/settings/general" activeProps={{ "aria-current": "page" }}>
            {intl.formatMessage({ id: "navigation.settings" })}
          </Link>
          <Link to="/trash" activeProps={{ "aria-current": "page" }}>
            {intl.formatMessage({ id: "navigation.trash" })}
          </Link>
          <Link to="/archive" activeProps={{ "aria-current": "page" }}>
            {intl.formatMessage({ id: "navigation.archive" })}
          </Link>
        </nav>
        <div className="presentation-controls">
          <Button
            className="locale"
            onPress={() => setLocale(locale === "en" ? "ar" : "en")}
          >
            {intl.formatMessage({
              id:
                locale === "en" ? "shell.localeArabic" : "shell.localeEnglish",
            })}
          </Button>
          <Button
            className="theme-toggle"
            onPress={() => setTheme(theme === "dark" ? "light" : "dark")}
          >
            {intl.formatMessage({ id: "shell.theme" })}
          </Button>
        </div>
      </aside>
      <section className="workspace">
        <header className="topbar">
          <Button onPress={() => setDialog("command")}>
            {intl.formatMessage({ id: "shell.search" })}{" "}
            <bdi className="shortcut">
              {intl.formatMessage({ id: "shell.searchShortcut" })}
            </bdi>
          </Button>
          <Button onPress={() => setDialog("capture")}>
            {intl.formatMessage({ id: "shell.capture" })}
          </Button>
          <Button onPress={() => setDialog("notifications")}>
            {intl.formatMessage({ id: "shell.notifications" })}
          </Button>
          <Button className="ai-trigger" onPress={() => setDialog("ai")}>
            {intl.formatMessage({ id: "shell.ai" })}
          </Button>
        </header>
        <Outlet />
        <DialogHost dialog={dialog} close={() => setDialog(null)} />
      </section>
    </main>
  );
}

function RoutePlaceholder() {
  const intl = useIntl();
  return (
    <section
      className="canvas feature-status"
      aria-labelledby="feature-status-heading"
    >
      <div className="eyebrow">
        {intl.formatMessage({ id: "status.foundation" })}
      </div>
      <h1 id="feature-status-heading">
        {intl.formatMessage({ id: "status.featureTitle" })}
      </h1>
      <p>{intl.formatMessage({ id: "status.featureDetail" })}</p>
    </section>
  );
}

function DialogHost({
  dialog,
  close,
}: {
  dialog: string | null;
  close: () => void;
}) {
  const intl = useIntl();
  if (!dialog) return null;
  const key =
    `dialog.${dialog}` as keyof typeof import("@lifeos/i18n/catalog").messages.en;
  return (
    <DialogTrigger isOpen onOpenChange={(open) => !open && close()}>
      <Button className="sr-only">
        {intl.formatMessage({ id: "dialog.open" })}
      </Button>
      <ModalOverlay className="overlay">
        <Modal className="modal">
          <Dialog>
            <Heading slot="title">{intl.formatMessage({ id: key })}</Heading>
            <p>
              {dialog === "ai"
                ? intl.formatMessage({ id: "status.offline" })
                : intl.formatMessage({ id: "dialog.placeholder" })}
            </p>
            <Button onPress={close}>
              {intl.formatMessage({ id: "common.cancel" })}
            </Button>
          </Dialog>
        </Modal>
      </ModalOverlay>
    </DialogTrigger>
  );
}
