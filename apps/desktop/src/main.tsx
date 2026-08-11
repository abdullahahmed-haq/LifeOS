import { StrictMode, useState } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { IntlProvider } from "react-intl";
import { messages, type Locale } from "@lifeos/i18n/catalog";
import { LifeOSRouter } from "./app/router";
import { PresentationProvider, usePresentation } from "./app/presentation";
import "./styles.css";

function LocalizedRouter() {
  const { locale } = usePresentation();
  return (
    <IntlProvider locale={locale} messages={messages[locale as Locale]}>
      <LifeOSRouter />
    </IntlProvider>
  );
}

export function App() {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: { queries: { retry: 1, staleTime: 5_000 } },
      }),
  );
  return (
    <QueryClientProvider client={queryClient}>
      <PresentationProvider>
        <LocalizedRouter />
      </PresentationProvider>
    </QueryClientProvider>
  );
}

const rootElement = document.getElementById("root");
if (rootElement) {
  createRoot(rootElement).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
