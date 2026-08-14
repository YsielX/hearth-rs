local function is_mech(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "mech" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_062",
    name = "Cobalt Guardian",
    text = "Whenever you summon a Mech, gain <b>Divine Shield</b>.",
    set = "GVG",
    type = "minion",
    class = "paladin",
    rarity = "rare",
    cost = 5,
    attack = 6,
    health = 3,
    tags = { "mech" },
    triggers = {
        {
            event = "minion_summoned",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.entity ~= self
                    and event.player == ctx:controller(self)
                    and is_mech(ctx, event.entity)
            end,
            effect = function(ctx, self)
                ctx:grant_keyword(self, "divine_shield")
            end,
        },
    },
}
