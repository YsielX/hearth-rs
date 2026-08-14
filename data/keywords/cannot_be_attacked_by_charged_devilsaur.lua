return {
    api_version = 1, module_type = "keyword", id = "cannot_be_attacked_by_charged_devilsaur",
    name = "Cannot Be Attacked By Charged Devilsaur",
    rules = {
        can_be_attacked = function(ctx, self, current, attacker)
            if attacker ~= nil and ctx:entity(attacker).card_id == "UNG_099"
                and ctx:get_data(attacker, "hero_attack_restricted") == 1 then return false end
            return current
        end,
    },
}
