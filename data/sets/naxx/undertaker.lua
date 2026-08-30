local function has_keyword(entity, wanted)
    for _, keyword in ipairs(entity.keywords) do
        if keyword == wanted then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "FP1_028",
    name = "Undertaker",
    text = "Whenever you summon a minion with <b>Deathrattle</b>, gain +1/+1.",
    set = "NAXX",
    type = "minion",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 2,
    tags = { "undead" },
    triggers = {
        {
            event = "minion_summoned",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and has_keyword(ctx:entity(event.entity), "deathrattle")
            end,
            effect = function(ctx, self, event)
                cardlib.effects.buff(ctx, self, 1, 1)
            end,
        },
    },
}
