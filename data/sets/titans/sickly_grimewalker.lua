local function is_undead(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "undead" or tag == "all" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "YOG_512",
    name = "Sickly Grimewalker",
    text = "[x]After you summon an\n Undead, give it <b>Poisonous</b>.",
    set = "TITANS",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    cost = 3,
    attack = 2,
    health = 4,
    rune_cost = { unholy = 1 },
    tags = { "undead" },
    triggers = {{
        event = "minion_summoned", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
                and event.entity ~= self
                and is_undead(ctx, event.entity)
        end,
        effect = function(ctx, self, event) ctx:grant_keyword(event.entity, "poisonous") end,
    }},
}
