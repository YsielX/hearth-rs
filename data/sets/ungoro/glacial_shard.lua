return {
    api_version = 1, id = "UNG_205", name = "Glacial Shard",
    text = "<b>Battlecry:</b> <b>Freeze</b> an enemy.",
    set = "UNGORO", type = "minion", rarity = "common", cost = 1, attack = 2, health = 1,
    tags = { "elemental" }, keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self) return ctx:enemy_characters(self) end,
    on_battlecry = function(ctx, self, target) if target then ctx:freeze(target) end end,
}
