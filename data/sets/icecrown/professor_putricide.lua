local function has_keyword(definition, wanted)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == wanted then return true end
    end
    return false
end

local card = {
    api_version = 1, id = "ICC_204", name = "Professor Putricide",
    text = "After you play a <b>Secret</b>,\nput a random Hunter <b>Secret</b> into the battlefield.",
    set = "ICECROWN", type = "minion", class = "hunter", rarity = "legendary",
    cost = 4, attack = 5, health = 4, tags = { "undead" },
}

card.triggers = {{
    event = "card_played", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event)
        if event.player ~= ctx:controller(self) or #ctx:secrets(event.player) >= 5 then return false end
        return has_keyword(ctx:card_definition(ctx:entity(event.entity).card_id), "secret")
    end,
    effect = function(ctx, self)
        local player = ctx:controller(self)
        local present, pool = {}, {}
        for _, secret in ipairs(ctx:secrets(player)) do present[ctx:entity(secret).card_id] = true end
        for _, id in ipairs(ctx:collectible_cards()) do
            local definition = ctx:card_definition(id)
            if definition.class == "hunter" and definition.type == "spell"
                and has_keyword(definition, "secret") and not present[id] then
                pool[#pool + 1] = id
            end
        end
        if #pool > 0 then ctx:random_value(pool, "putricide_secret_chosen") end
    end,
}}

function card.putricide_secret_chosen(ctx, self, card_id)
    ctx:cast_spell_if_valid(ctx:controller(self), card_id, nil)
end

return card
