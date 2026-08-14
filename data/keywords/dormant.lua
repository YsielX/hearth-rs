return {
    api_version = 1, module_type = "keyword", id = "dormant", name = "Dormant",
    rules = {
        can_attack = function(ctx, self, current) return false end,
        can_be_attacked = function(ctx, self, current) return false end,
        can_be_targeted = function(ctx, self, current) return false end,
    },
}
