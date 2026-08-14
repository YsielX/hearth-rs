return {
    api_version = 1,
    module_type = "keyword",
    id = "spell_damage",
    name = "Spell Damage",
    requires_param = true,

    rules = {
        base_spell_damage = function(ctx, self, current)
            local amount = ctx:keyword_param(self, "spell_damage")
            if amount == nil or amount < 0 then
                error("spell_damage keyword requires a non-negative numeric parameter")
            end
            return current + amount
        end,
    },
}
