// SPDX-License-Identifier: MIT
// Separate diagnostic: read the actual configured public spacing policy.
extern "C" unsigned bw_compaction_spacing() {
    return settings::getWStorage()->fs.minGCSpacing;
}
