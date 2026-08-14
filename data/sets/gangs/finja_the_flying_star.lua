local function is_murloc(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "murloc" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "CFM_344",
    name = "Finja, the Flying Star",
    text = "[x]<b>Stealth</b>\n   Whenever this attacks and   \nkills a minion, summon 2\n Murlocs from your deck.",
    set = "GANGS",
    type = "minion",
    rarity = "legendary",
    cost = 5,
    attack = 3,
    health = 5,
    tags = { "murloc" },
    keywords = { "stealth" },
    triggers = {{
        event = "attack", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.attacker == self
                and ctx:entity(event.defender).type == "minion"
                and ctx:entity(event.defender).health <= 0
        end,
        effect = function(ctx, self)
            ctx:set_data(self, "finja_remaining", 2)
            ctx:continue_with("recruit_next_murloc")
        end,
    }},
}

function card.recruit_next_murloc(ctx, self)
    local player = ctx:controller(self)
    if ctx:get_data(self, "finja_remaining") <= 0 or #ctx:board(player) >= 7 then return end
    local candidates = {}
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).type == "minion" and is_murloc(ctx, entity) then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "recruit_murloc") end
end

function card.recruit_murloc(ctx, self, entity)
    ctx:set_data(self, "finja_remaining", ctx:get_data(self, "finja_remaining") - 1)
    ctx:recruit(ctx:controller(self), entity)
    ctx:continue_with("recruit_next_murloc")
end

return card
