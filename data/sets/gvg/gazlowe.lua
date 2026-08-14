local card = {
    api_version = 1, id = "GVG_117", name = "Gazlowe",
    text = "Whenever you cast a 1-Cost spell, add a random Mech to your hand.", set = "GVG",
    type = "minion", rarity = "legendary", cost = 6, attack = 3, health = 6,
}
card.triggers = {{
    event = "spell_cast", active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and event.player_cast
            and ctx:entity(event.entity).cost == 1
    end,
    effect = function(ctx, self)
        local pool = {}
        for _, id in ipairs(ctx:collectible_cards()) do
            local definition = ctx:card_definition(id)
            if definition.type == "minion" then
                for _, tag in ipairs(definition.tags) do
                    if tag == "mech" then pool[#pool + 1] = id break end
                end
            end
        end
        if #pool > 0 then ctx:random_value(pool, "receive_mech") end
    end,
}}
function card.receive_mech(ctx, self, id) ctx:give_card(ctx:controller(self), id) end
return card
