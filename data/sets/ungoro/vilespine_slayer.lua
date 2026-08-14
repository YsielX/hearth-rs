return {
    api_version = 1, id = "UNG_064", name = "Vilespine Slayer",
    text = "<b>Combo:</b> Destroy a minion.",
    set = "UNGORO", type = "minion", class = "rogue", rarity = "epic",
    cost = 5, attack = 3, health = 4, keywords = { "combo" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        if not ctx:combo_active(self) then return {} end
        return ctx:minions()
    end,
    on_combo = function(ctx, self, target) if target then ctx:destroy(target) end end,
}
