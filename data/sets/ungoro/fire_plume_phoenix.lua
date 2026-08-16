return {
    api_version = 1, id = "UNG_084", name = "Fire Plume Phoenix",
    text = "<b>Battlecry:</b> Deal 3 damage.", set = "UNGORO", type = "minion",
    rarity = "common", cost = 4, attack = 3, health = 4, tags = { "elemental", "beast" },
    keywords = { "battlecry" }, target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
    on_battlecry = function(ctx, self, target) cardlib.effects.damage(ctx, target, 3) end,
}
