return {
    api_version = 1,
    module_type = "keyword",
    id = "cannot_be_attacked_by_fools_bane",
    name = "Cannot Be Attacked By Fools Bane",
    rules = {
        can_be_attacked = function(ctx, self, current, attacker)
            if attacker == nil or ctx:entity(attacker).type ~= "hero" then return current end
            local weapon = ctx:player(ctx:controller(attacker)).weapon
            if weapon and ctx:entity(weapon).card_id == "KAR_028" then return false end
            return current
        end,
    },
}
