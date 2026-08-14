local card = {
    api_version = 1, id = "AT_061", name = "Lock and Load",
    text = "Each time you cast a spell this turn, get a random Hunter card.",
    set = "TGT", type = "spell", class = "hunter", rarity = "epic", cost = 0,
}

function card.on_play(ctx, self) ctx:set_data(self, "active", 1) end

local function hunter_pool(ctx)
    local result = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        if definition.class == "hunter" and id ~= "AT_061" then result[#result + 1] = id end
    end
    return result
end

card.triggers = {
    {
        event = "card_played", timing = "before", active_zones = { "graveyard" },
        condition = function(ctx, self, event)
            return ctx:get_data(self, "active") == 1
                and event.player == ctx:controller(self)
                and event.entity ~= self
                and ctx:entity(event.entity).type == "spell"
        end,
        effect = function(ctx, self)
            local pool = hunter_pool(ctx)
            if #pool > 0 then ctx:random_value(pool, "receive_hunter_card") end
        end,
    },
    {
        event = "turn_ended", timing = "after", active_zones = { "graveyard" },
        condition = function(ctx, self, event)
            return ctx:get_data(self, "active") == 1
                and event.player == ctx:controller(self)
        end,
        effect = function(ctx, self) ctx:set_data(self, "active", 0) end,
    },
}

function card.receive_hunter_card(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
