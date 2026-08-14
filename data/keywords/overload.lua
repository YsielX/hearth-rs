return {
    api_version = 1,
    module_type = "keyword",
    id = "overload",
    name = "Overload",
    requires_param = true,

    hooks = {
        on_play = function(ctx, self)
            local amount = ctx:keyword_param(self, "overload")
            if amount == nil then
                error("overload keyword requires a numeric parameter")
            end
            if amount < 1 or amount > 255 then
                error("overload keyword parameter must be between 1 and 255")
            end
            ctx:overload(ctx:controller(self), amount)
        end,
    },
}
