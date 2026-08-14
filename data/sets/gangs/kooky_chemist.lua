return {
    api_version = 1,
    id = "CFM_063",
    name = "Kooky Chemist",
    text = "<b>Battlecry:</b> Swap the Attack and Health of a minion.",
    set = "GANGS",
    type = "minion",
    rarity = "common",
    cost = 4,
    attack = 4,
    health = 4,
    tags = { "undead" },
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx) return ctx:minions() end,
    on_battlecry = function(ctx, self, target)
        if not target then return end
        local entity = ctx:entity(target)
        ctx:modify(target, { stat = "attack", operation = "set", value = entity.health })
        ctx:set_health(target, entity.attack)
    end,
}
