return {
    api_version = 1,
    id = "EX1_124", rarity = "common",
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
        if not ctx:combo_active(self) then cardlib.effects.damage(ctx, target, 2) end
    end,
    on_combo = function(ctx, self, target) cardlib.effects.damage(ctx, target, 4) end,
}
