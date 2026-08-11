import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({ listAreas: vi.fn() }));

vi.mock("@lifeos/contracts/bindings", () => ({
  commands: {
    listAreas: mocks.listAreas,
    createArea: vi.fn(),
    undoAction: vi.fn()
  }
}));

import { App } from "./main";

describe("LifeOS application shell", () => {
  beforeEach(() => {
    localStorage.clear();
    mocks.listAreas.mockResolvedValue({ status: "ok", data: [] });
  });

  it("renders the empty Areas state and switches the document to Arabic RTL", async () => {
    const user = userEvent.setup();
    render(<App />);

    expect(await screen.findByRole("heading", { name: "No areas yet" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "العربية" }));

    expect(await screen.findByRole("heading", { name: "لا توجد مجالات بعد" })).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("ar");
    expect(document.documentElement.dir).toBe("rtl");
  });
});
