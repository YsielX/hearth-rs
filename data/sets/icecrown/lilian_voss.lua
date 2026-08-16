local card = {
    api_version = 1, id = "ICC_811", name = "Lilian Voss",
    text = "<b>Battlecry:</b> Replace spells in your hand with random spells <i>(from your opponent's class).</i>",
    set = "ICECROWN", type = "minion", class = "rogue", rarity = "legendary",
    cost = 4, attack = 4, health = 5, tags = { "undead" }, keywords = { "battlecry" },
}

local function spell_pool(ctx, self)
    local enemy_class = ctx:player(ctx:opponent(ctx:controller(self))).class
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        local eligible = definition.class == enemy_class
        for _, class in ipairs(definition.classes or {}) do
            if class == enemy_class then eligible = true; break end
        end
        if definition.type == "spell" and eligible then pool[#pool + 1] = id end
    end
    return pool
end

function card.on_battlecry(ctx, self) ctx:continue_with("replace_next_voss_spell") end

function card.replace_next_voss_spell(ctx, self)
    local target = nil
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "spell" and ctx:get_data(self, "voss_done:" .. entity) == 0 then
            target = entity
            break
        end
    end
    local pool = spell_pool(ctx, self)
    if target ~= nil and #pool > 0 then
        ctx:set_data(self, "voss_target", target)
        ctx:random_value(pool, "transform_voss_spell")
    end
end

function card.transform_voss_spell(ctx, self, card_id)
    local target = ctx:get_data(self, "voss_target")
    if target > 0 and ctx:entity(target).zone == "hand" then
        cardlib.effects.transform(ctx, target, card_id)
        ctx:set_data(self, "voss_done:" .. target, 1)
    end
    ctx:continue_with("replace_next_voss_spell")
end

return card
