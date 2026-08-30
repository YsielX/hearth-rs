local card = {
    api_version = 1, id = "CFM_781", name = "Shaku, the Collector",
    text = "[x]<b>Stealth</b>\nWhenever this attacks,\nadd a card from another\nclass to your hand.",
    set = "GANGS", type = "minion", class = "rogue", rarity = "legendary",
    cost = 3, attack = 2, health = 4, keywords = { "stealth" },
}
card.triggers = {{
    event = "attack", timing = "before", active_zones = { "board" },
    condition = function(ctx, self, event) return event.attacker == self end,
    effect = function(ctx, self)
        local own_class, pool = ctx:player(ctx:controller(self)).class, {}
        for _, id in ipairs(ctx:collectible_cards()) do
            local definition = ctx:card_definition(id)
            local eligible = definition.class ~= "neutral" and definition.class ~= own_class
            if definition.classes and #definition.classes > 0 then
                eligible = true
                for _, class in ipairs(definition.classes) do if class == own_class then eligible = false end end
            end
            if eligible then pool[#pool + 1] = id end
        end
        if #pool > 0 then ctx:random_value(pool, "add_shaku_card") end
    end,
}}
function card.add_shaku_card(ctx, self, id) cardlib.effects.give_card(ctx, ctx:controller(self), id) end
return card
