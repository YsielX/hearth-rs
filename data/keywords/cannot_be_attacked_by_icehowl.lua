return {
    api_version = 1, module_type = "keyword", id = "cannot_be_attacked_by_icehowl",
    name = "Cannot Be Attacked By Icehowl",
    rules = {
        can_be_attacked = function(ctx, self, current, attacker)
            if attacker ~= nil and ctx:entity(attacker).card_id == "AT_125" then return false end
            return current
        end,
    },
}
