#!/usr/bin/env node

const { chromium } = require("playwright");

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const parsed = {
    roomUrl: null,
    botName: "Recorder",
    display: ":99",
    timeout: 0,
  };

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case "--room-url":
        parsed.roomUrl = args[++i];
        break;
      case "--bot-name":
        parsed.botName = args[++i];
        break;
      case "--display":
        parsed.display = args[++i];
        break;
      case "--timeout":
        parsed.timeout = parseInt(args[++i], 10);
        break;
    }
  }

  if (!parsed.roomUrl) {
    console.error("Error: --room-url is required");
    process.exit(1);
  }

  return parsed;
}

// ---------------------------------------------------------------------------
// BBB join flow
// ---------------------------------------------------------------------------

async function joinMeeting(page, roomUrl, botName, timeout) {
  const deadline = timeout > 0 ? Date.now() + timeout * 1000 : Date.now() + 600_000;

  while (Date.now() < deadline) {
    // Wait for page to settle after navigation
    await page.waitForLoadState("networkidle", { timeout: 30_000 });

    // Step 1: Fill in the username if prompted (Greenlight / BBB join page)
    const nameInput = page.locator(
      'input#join_name, input#joinFormName, input[name="username"], input[id="username"], input[name="name"][placeholder="Enter your name"]'
    );
    try {
      await nameInput.first().waitFor({ state: "visible", timeout: 10_000 });
      await nameInput.first().fill(botName);
      // Click the join / submit button
      const joinBtn = page.locator(
        'button[type="submit"], input[type="submit"], button:has-text("Join")'
      );
      await joinBtn.first().click();
      console.error(`Filled name "${botName}" and clicked join`);
    } catch {
      // No name prompt — might already be in the meeting or auto-joined
      console.error("No name input found, proceeding");
    }

    // Step 2: Wait for the audio modal to confirm we're in the meeting.
    // If the meeting isn't active, Greenlight shows a spinner instead of
    // redirecting to BBB — the audio modal will never appear.
    const listenOnly = page.locator(
      'button[aria-label="Listen only"], button:has-text("Listen only"), button[data-test="listenOnlyBtn"]'
    );
    try {
      await listenOnly.first().waitFor({ state: "visible", timeout: 15_000 });
      await listenOnly.first().click();
      console.error("Clicked Listen only — meeting is active");

      // Step 3: Dismiss any remaining modals (e.g. welcome message, notifications)
      const closeButtons = page.locator(
        'button[aria-label="Close"], button[data-test="closeModal"], button:has-text("OK")'
      );
      try {
        const count = await closeButtons.count();
        for (let i = 0; i < count; i++) {
          if (await closeButtons.nth(i).isVisible()) {
            await closeButtons.nth(i).click();
            console.error("Dismissed a modal");
          }
        }
      } catch {
        // No modals to dismiss
      }

      // Step 4: Wait for meeting content to be visible
      const meetingContent = page.locator(
        '[data-test="presentationContainer"], [class*="presentation"], video, canvas'
      );
      try {
        await meetingContent.first().waitFor({ state: "visible", timeout: 30_000 });
        console.error("Meeting content visible");
      } catch {
        console.error(
          "Warning: Could not detect meeting content, continuing anyway"
        );
      }

      return;
    } catch {
      // Audio modal didn't appear — meeting likely not active yet
      console.error("Meeting not active yet, retrying in 10s...");
      await new Promise((r) => setTimeout(r, 10_000));
      await page.goto(roomUrl, {
        waitUntil: "domcontentloaded",
        timeout: 30_000,
      });
    }
  }

  throw new Error("Timed out waiting for meeting to become active");
}

// ---------------------------------------------------------------------------
// Meeting end detection
// ---------------------------------------------------------------------------

function watchForMeetingEnd(page, bbbClientUrlPattern) {
  return new Promise((resolve) => {
    // Also detect navigation away from the BBB html5 client (e.g. redirect
    // back to Greenlight when the meeting ends).
    const onNavigate = (frame) => {
      if (frame === page.mainFrame()) {
        const url = frame.url();
        if (bbbClientUrlPattern && !url.includes("html5client")) {
          console.error(`Navigated away from BBB client to ${url}`);
          page.removeListener("framenavigated", onNavigate);
          resolve();
        }
      }
    };
    page.on("framenavigated", onNavigate);

    const check = async () => {
      try {
        const ended = await page
          .locator(
            'text="The meeting has ended", text="This meeting has ended", text="This session has ended", text="You have been logged out", text="Die Konferenz wurde beendet", text="Diese Sitzung wurde beendet", [data-test="meetingEndedModal"]'
          )
          .first()
          .isVisible();
        if (ended) {
          page.removeListener("framenavigated", onNavigate);
          resolve();
          return;
        }
      } catch {
        // Page might be closed
        page.removeListener("framenavigated", onNavigate);
        resolve();
        return;
      }
      setTimeout(check, 5_000);
    };
    setTimeout(check, 5_000);
  });
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main() {
  const opts = parseArgs();
  let browser = null;
  let shutdownRequested = false;

  async function shutdown() {
    if (shutdownRequested) return;
    shutdownRequested = true;
    console.log("RECORDING_STOPPED");
    if (browser) {
      try {
        await browser.close();
      } catch {
        // Browser may already be closed
      }
    }
    process.exit(0);
  }

  process.on("SIGTERM", shutdown);
  process.on("SIGINT", shutdown);

  // Set DISPLAY for Chromium to render on the virtual framebuffer
  // Unset Wayland so Chromium uses X11 (required for Xvfb capture)
  process.env.DISPLAY = opts.display;
  delete process.env.WAYLAND_DISPLAY;

  try {
    browser = await chromium.launch({
      headless: false,
      args: [
        "--no-sandbox",
        "--disable-dev-shm-usage",
        "--autoplay-policy=no-user-gesture-required",
        "--disable-background-timer-throttling",
        "--disable-backgrounding-occluded-windows",
        "--disable-renderer-backgrounding",
        "--ozone-platform=x11",
        "--start-fullscreen",
        "--window-size=1920,1080",
        "--window-position=0,0",
      ],
    });

    const context = await browser.newContext({
      viewport: { width: 1920, height: 1080 },
      ignoreHTTPSErrors: true,
      permissions: ["microphone", "camera"],
    });

    const page = await context.newPage();

    console.error(`Navigating to ${opts.roomUrl}`);
    await page.goto(opts.roomUrl, {
      waitUntil: "domcontentloaded",
      timeout: 30_000,
    });

    await joinMeeting(page, opts.roomUrl, opts.botName, opts.timeout);

    // Signal to the orchestrator that the browser is ready
    console.log("RECORDING_STARTED");

    // Wait for shutdown signal, timeout, or meeting end
    const promises = [watchForMeetingEnd(page, true)];

    if (opts.timeout > 0) {
      promises.push(
        new Promise((resolve) => setTimeout(resolve, opts.timeout * 1000))
      );
    }

    await Promise.race(promises);
    console.error("Meeting ended or timeout reached");
    await shutdown();
  } catch (err) {
    console.error(`Fatal error: ${err.message}`);
    if (browser) {
      try {
        await browser.close();
      } catch {
        // ignore
      }
    }
    process.exit(1);
  }
}

main();
