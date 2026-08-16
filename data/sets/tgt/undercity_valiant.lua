return {
    api_version = 1,
    id = "AT_030",
    name = "Undercity Valiant",
    text = "<b>Combo:</b> Deal 1 damage.",
    set = "TGT",
    type = "minion",
    class = "rogue",
    rarity = "common",
    cost = 2,
    attack = 3,
    health = 2,
    tags = { "undead" },
    keywords = { "combo" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        if ctx:combo_active(self) then return ctx:characters() end
        return {}
    end,
    on_combo = function(ctx, self, target)
        if target ~= nil then cardlib.effects.damage(ctx, target, 1) end
    end,
}
