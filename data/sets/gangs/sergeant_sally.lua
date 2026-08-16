return {
    api_version = 1,
    id = "CFM_341",
    name = "Sergeant Sally",
    text = "<b>Deathrattle:</b> Deal damage equal to this minion's Attack to all enemy minions.",
    set = "GANGS",
    type = "minion",
    rarity = "legendary",
    cost = 3,
    attack = 1,
    health = 1,
    tags = { "undead" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local me = ctx:entity(self)
        local attack = math.max(0, me.attack_at_death or me.attack or 0)
        local targets = {}
        for _, target in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(target).type == "minion" then targets[#targets + 1] = target end
        end
        if #targets > 0 then cardlib.effects.damage_all(ctx, targets, attack) end
    end,
}
