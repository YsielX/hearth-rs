local function is_pirate(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "pirate" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_025",
    name = "One-eyed Cheat",
    text = "Whenever you summon a Pirate, gain <b>Stealth</b>.",
    set = "GVG",
    type = "minion",
    class = "rogue",
    rarity = "rare",
    cost = 2,
    attack = 4,
    health = 1,
    tags = { "pirate" },
    triggers = {
        {
            event = "minion_summoned",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.entity ~= self
                    and event.player == ctx:controller(self)
                    and is_pirate(ctx, event.entity)
            end,
            effect = function(ctx, self)
                ctx:grant_keyword(self, "stealth")
            end,
        },
    },
}
