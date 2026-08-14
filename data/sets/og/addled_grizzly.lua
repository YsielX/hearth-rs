local function beast(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "beast" or tag == "all" then return true end
    end
    return false
end
return {
    api_version = 1, id = "OG_313", name = "Addled Grizzly",
    text = "After you summon a\nBeast, give it +1/+1.",
    set = "OG", type = "minion", class = "druid", rarity = "rare",
    cost = 2, attack = 2, health = 3, tags = { "beast" },
    triggers = {{
        event = "minion_summoned", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.entity ~= self
                and beast(ctx, event.entity)
        end,
        effect = function(ctx, self, event) ctx:buff(event.entity, 1, 1) end,
    }},
}
