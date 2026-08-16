local card = {
    api_version = 1, id = "CFM_697", name = "Lotus Illusionist",
    text = "[x]After this minion attacks\na hero, transform it into a\n random 6-Cost minion.",
    set = "GANGS", type = "minion", class = "shaman", rarity = "epic",
    cost = 4, attack = 3, health = 5,
    triggers = {{
        event = "attack", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.attacker == self and ctx:entity(event.defender).type == "hero"
        end,
        effect = function(ctx, self)
            local pool = {}
            for _, card_id in ipairs(ctx:collectible_cards()) do
                local definition = ctx:card_definition(card_id)
                if definition.type == "minion" and definition.cost == 6 then pool[#pool + 1] = card_id end
            end
            if #pool > 0 then ctx:random_value(pool, "become_minion") end
        end,
    }},
}
function card.become_minion(ctx, self, card_id) cardlib.effects.transform(ctx, self, card_id) end
return card
