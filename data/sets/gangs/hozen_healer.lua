return {
    api_version = 1,
    id = "CFM_067",
    name = "Hozen Healer",
    text = "<b>Battlecry</b>: Restore a minion to full Health.",
    set = "GANGS",
    type = "minion",
    rarity = "common",
    cost = 4,
    attack = 2,
    health = 6,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx) return ctx:minions() end,
    on_battlecry = function(ctx, self, target)
        if not target then return end
        local entity = ctx:entity(target)
        cardlib.effects.heal(ctx, target, entity.max_health - entity.health)
    end,
}
