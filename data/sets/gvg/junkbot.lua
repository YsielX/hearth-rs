local function is_mech(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "mech" then return true end
    end
    return false
end
return {
    api_version = 1, id = "GVG_106", name = "Junkbot",
    text = "Whenever a friendly Mech dies, gain +2/+2.", set = "GVG", type = "minion",
    rarity = "epic", cost = 5, attack = 1, health = 5, tags = { "mech" },
    triggers = {{
        event = "entity_died", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.entity ~= self and is_mech(ctx, event.entity)
        end,
        effect = function(ctx, self) cardlib.effects.buff(ctx, self, 2, 2) end,
    }},
}
