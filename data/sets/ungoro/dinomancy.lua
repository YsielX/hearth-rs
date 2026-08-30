local function is_beast(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "beast" or tag == "all" then return true end
    end
    return false
end
return { api_version = 1, id = "UNG_917", name = "Dinomancy",
    text = "Replace your Hero Power with 'Give a Beast +3/+3.'", set = "UNGORO",
    type = "spell", class = "hunter", rarity = "epic", cost = 2,
    on_play = function(ctx, self) ctx:replace_hero_power(ctx:controller(self), "UNG_917t1") end,
    tokens = {{ id = "UNG_917t1", name = "Dinomancy", text = "Give a Beast +3/+3.", set = "UNGORO",
        type = "hero_power", class = "hunter", cost = 2, target_mode = "required",
        targets = function(ctx, self)
            local result = {}
            for _, minion in ipairs(ctx:friendly_minions(self)) do if is_beast(ctx, minion) then result[#result + 1] = minion end end
            return result
        end,
        on_play = function(ctx, self, target) cardlib.effects.buff(ctx, target, 3, 3) end }} }
