import { describe, expect, test } from "vitest";
import { footprintForPresentation, shouldApplyHoverChange } from "./quickWindowFootprint";

describe("quick window footprint", () => {
  test("transparent area collapses when speech is hidden", () => {
    expect(footprintForPresentation({ bubbleVisible: false })).toBe("compact");
  });

  test("speech gets the expanded presentation area only while visible", () => {
    expect(footprintForPresentation({ bubbleVisible: true })).toBe("expanded");
  });

  test("resize-generated pointer leave cannot immediately collapse speech", () => {
    expect(shouldApplyHoverChange(false, true)).toBe(false);
    expect(shouldApplyHoverChange(false, false)).toBe(true);
    expect(shouldApplyHoverChange(true, true)).toBe(true);
  });
});
