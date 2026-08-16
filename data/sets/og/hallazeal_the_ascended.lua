local function first_hallazeal(ctx, self)
    for _, minion in ipairs(ctx:board(ctx:controller(self))) do
        local entity = ctx:entity(minion)
        local dormant = false
        for _, keyword in ipairs(entity.keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if entity.card_id == "OG_209" and minion < self
            and not entity.silenced and not dormant then return false end
    end
    return true
end

return {
    api_version = 1, id = "OG_209", name = "Hallazeal the Ascended",
    text = "<b>Spell Damage +1</b>\nYour spells have <b>Lifesteal</b>.", set = "OG",
    type = "minion", class = "shaman", rarity = "legendary", cost = 5, attack = 4, health = 6,
    tags = { "elemental" }, keywords = { "spell_damage" }, keyword_params = { spell_damage = 1 },
    triggers = {{
        event = "damaged", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            if event.amount <= 0 or not first_hallazeal(ctx, self) then return false end
            local source = ctx:entity(event.source)
            return source.type == "spell" and source.controller == ctx:controller(self)
        end,
        effect = function(ctx, self, event)
            local player = ctx:controller(self)
            cardlib.effects.heal(ctx, ctx:player(player).hero, event.amount)
        end,
    }},
}
