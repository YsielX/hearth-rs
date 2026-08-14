return {
    api_version = 1,
    id = "EX1_124",
    name = "Eviscerate",
    text = "Deal $2 damage. <b>Combo:</b> Deal $4 damage instead.",
    set = "EXPERT1",
    type = "spell",
    class = "rogue",
    cost = 2,
    target_mode = "required",
    keywords = { "combo" },
    targets = function(ctx, self) return ctx:enemy_characters(self) end,
    on_play = function(ctx, self, target)
        if not ctx:combo_active(self) then ctx:damage(target, 2) end
    end,
    on_combo = function(ctx, self, target) ctx:damage(target, 4) end,
}
